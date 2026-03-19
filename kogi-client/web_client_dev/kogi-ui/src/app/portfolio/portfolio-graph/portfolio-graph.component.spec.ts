import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PortfolioGraphComponent } from './portfolio-graph.component';

describe('PortfolioGraphComponent', () => {
  let component: PortfolioGraphComponent;
  let fixture: ComponentFixture<PortfolioGraphComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [PortfolioGraphComponent]
    }).compileComponents();

    fixture = TestBed.createComponent(PortfolioGraphComponent);
    component = fixture.componentInstance;
    await fixture.whenStable();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
