import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PortfolioFolderComponent } from './portfolio-folder.component';

describe('PortfolioFolderComponent', () => {
  let component: PortfolioFolderComponent;
  let fixture: ComponentFixture<PortfolioFolderComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [PortfolioFolderComponent]
    }).compileComponents();

    fixture = TestBed.createComponent(PortfolioFolderComponent);
    component = fixture.componentInstance;
    await fixture.whenStable();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
